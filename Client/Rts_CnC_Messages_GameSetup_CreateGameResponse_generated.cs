using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_GameSetup_CreateGameResponse
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.GameSetup.CreateGameResponse); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.GameSetup.CreateGameResponse)obj;
            //  Serialize RequestId
            s.Write(ref value.RequestId);
            //  Serialize Succeeded
            s.Write(value.Succeeded);
            //  Serialize array SecurityTokens
            Rts.Serialization.Reference.Write(s, value.SecurityTokens, () =>
            {
                s.WriteVarInt32(value.SecurityTokens.Length);
                for(int i = 0 ; i < value.SecurityTokens.Length ; ++i)
                {
                    GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_GameSetup_CreateGameResponse_Element.Serializer.Serialize(s, value.SecurityTokens[i]);
                }
            });

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.GameSetup.CreateGameResponse)) as Rts.CnC.Messages.GameSetup.CreateGameResponse;
            //  Deserialize RequestId
            s.Read(out value.RequestId);
            //  Deserialize Succeeded
            s.Read(out value.Succeeded);
            //  Deserialize array SecurityTokens
            Rts.Serialization.Reference.Read(s, out value.SecurityTokens, () =>
            {
                int length = s.ReadVarInt32();
                Rts.CnC.Messages.GameSetup.CreateGameResponse.Element[] tmp = new Rts.CnC.Messages.GameSetup.CreateGameResponse.Element[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_GameSetup_CreateGameResponse_Element.Serializer.DeserializeValue(s, ref tmp[i]);
                }
                return tmp;
            });

            return value;
        }
        
    }
}
