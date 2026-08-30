using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_DevDB_SelectServerSettingsMsg
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.DevDB.SelectServerSettingsMsg); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.DevDB.SelectServerSettingsMsg)obj;
            //  Serialize ConfigurationName
            s.Write(value.ConfigurationName);
            //  Serialize ServerType
            s.Write(value.ServerType);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.DevDB.SelectServerSettingsMsg)) as Rts.CnC.Messages.DevDB.SelectServerSettingsMsg;
            //  Deserialize ConfigurationName
            s.Read(out value.ConfigurationName);
            //  Deserialize ServerType
            s.Read(out value.ServerType);

            return value;
        }
        
    }
}
