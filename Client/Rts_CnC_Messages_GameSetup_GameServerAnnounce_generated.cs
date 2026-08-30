using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_GameSetup_GameServerAnnounce
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.GameSetup.GameServerAnnounce); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.GameSetup.GameServerAnnounce)obj;
            //  Serialize GameServerMessagingAddress
            s.Write(value.GameServerMessagingAddress);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.GameSetup.GameServerAnnounce)) as Rts.CnC.Messages.GameSetup.GameServerAnnounce;
            //  Deserialize GameServerMessagingAddress
            s.Read(out value.GameServerMessagingAddress);

            return value;
        }
        
    }
}
