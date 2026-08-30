using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_DevDB_SelectServerSettingsResponseMsg
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.DevDB.SelectServerSettingsResponseMsg); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.DevDB.SelectServerSettingsResponseMsg)obj;
            //  Serialize Settings
            s.Write(value.Settings);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.DevDB.SelectServerSettingsResponseMsg)) as Rts.CnC.Messages.DevDB.SelectServerSettingsResponseMsg;
            //  Deserialize Settings
            s.Read(out value.Settings);

            return value;
        }
        
    }
}
