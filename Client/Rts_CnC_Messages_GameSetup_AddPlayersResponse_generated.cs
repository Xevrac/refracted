using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_GameSetup_AddPlayersResponse
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.GameSetup.AddPlayersResponse); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.GameSetup.AddPlayersResponse)obj;
            //  Serialize RequestId
            s.Write(ref value.RequestId);
            //  Serialize array Responses
            Rts.Serialization.Reference.Write(s, value.Responses, () =>
            {
                s.WriteVarInt32(value.Responses.Length);
                for(int i = 0 ; i < value.Responses.Length ; ++i)
                {
                    GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_GameSetup_AddPlayersResponse_Element.Serializer.Serialize(s, value.Responses[i]);
                }
            });

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.GameSetup.AddPlayersResponse)) as Rts.CnC.Messages.GameSetup.AddPlayersResponse;
            //  Deserialize RequestId
            s.Read(out value.RequestId);
            //  Deserialize array Responses
            Rts.Serialization.Reference.Read(s, out value.Responses, () =>
            {
                int length = s.ReadVarInt32();
                Rts.CnC.Messages.GameSetup.AddPlayersResponse.Element[] tmp = new Rts.CnC.Messages.GameSetup.AddPlayersResponse.Element[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_GameSetup_AddPlayersResponse_Element.Serializer.DeserializeValue(s, ref tmp[i]);
                }
                return tmp;
            });

            return value;
        }
        
    }
}
