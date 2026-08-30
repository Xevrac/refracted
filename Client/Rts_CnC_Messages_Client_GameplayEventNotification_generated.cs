using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_GameplayEventNotification
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.GameplayEventNotification); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.GameplayEventNotification)obj;
            //  Serialize EventLocation
            s.Write(value.EventLocation);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.GameplayEventNotification)) as Rts.CnC.Messages.Client.GameplayEventNotification;
            //  Deserialize EventLocation
            s.Read(out value.EventLocation);

            return value;
        }
        
    }
}
