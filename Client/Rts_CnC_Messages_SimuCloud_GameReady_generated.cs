using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_SimuCloud_GameReady
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.SimuCloud.GameReady); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.SimuCloud.GameReady)obj;
            //  Serialize Server
            s.Write(ref value.Server);
            //  Serialize Game
            s.Write(ref value.Game);
            //  Serialize array SecurityTokens
            Rts.Serialization.Reference.Write(s, value.SecurityTokens, () =>
            {
                s.WriteVarInt32(value.SecurityTokens.Length);
                for(int i = 0 ; i < value.SecurityTokens.Length ; ++i)
                {
                    GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_SimuCloud_GameReady_Element.Serializer.Serialize(s, value.SecurityTokens[i]);
                }
            });

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.SimuCloud.GameReady)) as Rts.CnC.Messages.SimuCloud.GameReady;
            //  Deserialize Server
            s.Read(out value.Server);
            //  Deserialize Game
            s.Read(out value.Game);
            //  Deserialize array SecurityTokens
            Rts.Serialization.Reference.Read(s, out value.SecurityTokens, () =>
            {
                int length = s.ReadVarInt32();
                Rts.CnC.Messages.SimuCloud.GameReady.Element[] tmp = new Rts.CnC.Messages.SimuCloud.GameReady.Element[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_SimuCloud_GameReady_Element.Serializer.DeserializeValue(s, ref tmp[i]);
                }
                return tmp;
            });

            return value;
        }
        
    }
}
